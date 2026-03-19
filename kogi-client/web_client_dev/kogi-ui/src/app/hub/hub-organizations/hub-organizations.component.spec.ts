import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubOrganizationsComponent } from './hub-organizations.component';

describe('HubOrganizationsComponent', () => {
  let component: HubOrganizationsComponent;
  let fixture: ComponentFixture<HubOrganizationsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubOrganizationsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubOrganizationsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
