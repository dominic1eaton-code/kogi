import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubGovernanceComponent } from './hub-governance.component';

describe('HubGovernanceComponent', () => {
  let component: HubGovernanceComponent;
  let fixture: ComponentFixture<HubGovernanceComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubGovernanceComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubGovernanceComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
