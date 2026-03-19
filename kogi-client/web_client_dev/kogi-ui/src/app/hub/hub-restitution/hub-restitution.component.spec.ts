import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubRestitutionComponent } from './hub-restitution.component';

describe('HubRestitutionComponent', () => {
  let component: HubRestitutionComponent;
  let fixture: ComponentFixture<HubRestitutionComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubRestitutionComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubRestitutionComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
